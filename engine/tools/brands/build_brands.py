#!/usr/bin/env python3
"""Builds engine/predict/brands.tsv (docs/moonshot.md §5.4) from Wikidata (CC0).

    python3 -I engine/tools/brands/build_brands.py [--min-sitelinks N]

Companies and brands with an English name and a Korean or Japanese one, and their country, kept
when they are in a major stock index (or big enough to be) or known by their name alone
(admission.py, the owner's bar of 2026-11-05). Each
line: the brand's default name, then every spelling it is typed as (normalized: Korean and kana
without spaces, kana as katakana, English lower-cased letters and digits only). The default name
follows the country of origin (docs/moonshot.md §5.4): a name ending in "Korea" is Korean, one
ending in "Japan" English; else a Korean company Korean, a Japanese one Japanese, any other
English. The raw query results are cached in corpora/brands/ (gitignored), with admission.tsv:
every candidate and why it was kept or left out.
"""

import argparse
import json
import os
import re
import sys
import time
import unicodedata
import urllib.parse
import urllib.request

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import admission  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CACHE = os.path.join(ROOT, "corpora", "brands")
OUT = os.path.join(ROOT, "engine", "predict", "brands.tsv")
ENDPOINT = "https://query.wikidata.org/sparql"
AGENT = "jis-moonshot-brands/0.2 (https://github.com/team-mirae/jis-moonshot)"

# Kinds of company or brand, queried one at a time so no query times out.
CLASSES = {
    "Q4830453": "business",
    "Q783794": "company",
    "Q891723": "public company",
    "Q6881511": "enterprise",
    "Q431289": "brand",
    "Q167037": "corporation",
    "Q658255": "subsidiary",
    "Q778575": "conglomerate",
    "Q219577": "holding company",
    "Q18388277": "technology company",
    "Q1058914": "software company",
    "Q786820": "automobile manufacturer",
    "Q210167": "video game developer",
    "Q1137109": "video game publisher",
    "Q507619": "retail chain",
    "Q18534": "restaurant chain",
    "Q46970": "airline",
    "Q22687": "bank",
    "Q1941618": "mobile network operator",
    "Q166280": "kabushiki gaisha",
    "Q134161": "joint-stock company",
    "Q2085381": "publisher",
    "Q1762059": "film production company",
    "Q18127": "record label",
    "Q15265344": "broadcaster",
    "Q1320047": "pharmaceutical company",
    "Q11032": "newspaper",
}

# Anything with an industry (P452) is a company or a brand, whatever its class: in slices of
# sitelinks so no query times out.
INDUSTRY = """
SELECT ?item ?links ?country ?en ?ko ?ja ?kana
       (GROUP_CONCAT(DISTINCT ?koAlt; separator="|") AS ?koAlts)
       (GROUP_CONCAT(DISTINCT ?jaAlt; separator="|") AS ?jaAlts)
       (GROUP_CONCAT(DISTINCT ?enAlt; separator="|") AS ?enAlts) WHERE {
  ?item wdt:P452 [] ; wikibase:sitelinks ?links .
  FILTER(?links >= %(lo)d && ?links < %(hi)d)
  ?item rdfs:label ?en . FILTER(LANG(?en) = "en")
  OPTIONAL { ?item rdfs:label ?ko . FILTER(LANG(?ko) = "ko") }
  OPTIONAL { ?item rdfs:label ?ja . FILTER(LANG(?ja) = "ja") }
  FILTER(BOUND(?ko) || BOUND(?ja))
  OPTIONAL { ?item wdt:P1814 ?kana }
  OPTIONAL { ?item (wdt:P17|wdt:P495) ?c . ?c wdt:P297 ?country }
  OPTIONAL { ?item skos:altLabel ?koAlt . FILTER(LANG(?koAlt) = "ko") }
  OPTIONAL { ?item skos:altLabel ?jaAlt . FILTER(LANG(?jaAlt) = "ja") }
  OPTIONAL { ?item skos:altLabel ?enAlt . FILTER(LANG(?enAlt) = "en") }
}
GROUP BY ?item ?links ?country ?en ?ko ?ja ?kana
"""
SLICES = [(8, 10), (10, 13), (13, 17), (17, 23), (23, 32), (32, 50), (50, 100), (100, 100000)]

QUERY = """
SELECT ?item ?links ?country ?en ?ko ?ja ?kana
       (GROUP_CONCAT(DISTINCT ?koAlt; separator="|") AS ?koAlts)
       (GROUP_CONCAT(DISTINCT ?jaAlt; separator="|") AS ?jaAlts)
       (GROUP_CONCAT(DISTINCT ?enAlt; separator="|") AS ?enAlts) WHERE {
  ?item wdt:P31 wd:%(cls)s ; wikibase:sitelinks ?links .
  FILTER(?links >= %(min)d)
  ?item rdfs:label ?en . FILTER(LANG(?en) = "en")
  OPTIONAL { ?item rdfs:label ?ko . FILTER(LANG(?ko) = "ko") }
  OPTIONAL { ?item rdfs:label ?ja . FILTER(LANG(?ja) = "ja") }
  FILTER(BOUND(?ko) || BOUND(?ja))
  OPTIONAL { ?item wdt:P1814 ?kana }
  OPTIONAL { ?item (wdt:P17|wdt:P495) ?c . ?c wdt:P297 ?country }
  OPTIONAL { ?item skos:altLabel ?koAlt . FILTER(LANG(?koAlt) = "ko") }
  OPTIONAL { ?item skos:altLabel ?jaAlt . FILTER(LANG(?jaAlt) = "ja") }
  OPTIONAL { ?item skos:altLabel ?enAlt . FILTER(LANG(?enAlt) = "en") }
}
GROUP BY ?item ?links ?country ?en ?ko ?ja ?kana
"""


def fetch(name, query):
    os.makedirs(CACHE, exist_ok=True)
    path = os.path.join(CACHE, f"{name}.json")
    if os.path.exists(path):
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    url = ENDPOINT + "?" + urllib.parse.urlencode({"query": query, "format": "json"})
    for attempt in range(3):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": AGENT, "Accept": "application/sparql-results+json"})
            with urllib.request.urlopen(req, timeout=90) as r:
                data = json.load(r)["results"]["bindings"]
            with open(path, "w", encoding="utf-8") as f:
                json.dump(data, f, ensure_ascii=False)
            return data
        except Exception as e:  # noqa: BLE001 - a timeout or rate limit: try again, then skip
            print(f"  {name}: {e}; retrying", file=sys.stderr)
            time.sleep(5 * (attempt + 1))
    return []


def katakana(s):
    return "".join(chr(ord(c) + 0x60) if "ぁ" <= c <= "ゖ" else c for c in s)


def norm_cjk(s):
    s = unicodedata.normalize("NFKC", s)
    s = re.sub(r"\s+|・|·|\.", "", s)
    return katakana(s)


def norm_en(s):
    s = unicodedata.normalize("NFKD", s)
    return re.sub(r"[^0-9a-z]", "", s.lower())


LEGAL = re.compile(
    r"(,?\s+(Inc|Incorporated|Corp|Corporation|Co|Company|Ltd|Limited|LLC|L\.L\.C|PLC|plc|AG|SE|SA|S\.A|NV|N\.V|"
    r"GmbH|KK|K\.K|Co\.,?\s*Ltd|Holdings?)\.?)+$"
    r"|^(주식회사|\(주\)|㈜)\s*|\s*(주식회사|\(주\)|㈜)$|^(株式会社|\(株\)|㈱)\s*|\s*(株式会社|\(株\)|㈱)$"
)


def plain(name):
    """A name without its legal form (Apple Inc. → Apple, 네이버 주식회사 → 네이버)."""
    stripped = LEGAL.sub("", name).strip()
    return stripped or name


# Corporate words a short form leaves off (소니 for 소니 그룹, ホンダ for ホンダ技研工業): stripped
# for matching only.
CORPORATE = re.compile(
    r"(그룹|홀딩스|주식회사|코퍼레이션|컴퍼니|기술공업|공업|자동차|전자|グループ|ホールディングス|株式会社|自動車|技研工業|工業|製作所|"
    r"group|holdings?|corporation|company|motors?|electronics)$"
)


def writes(name, script):
    """Whether `name` is written in `script`: Hangul, kana or kanji, or Latin only."""
    if script == "ko":
        return any("\uac00" <= c <= "\ud7a3" for c in name)
    if script == "ja":
        return any("\u3040" <= c <= "\u30ff" or "\u4e00" <= c <= "\u9fff" for c in name) and not any("\uac00" <= c <= "\ud7a3" for c in name)
    return all(ord(c) < 0x2000 for c in name)


def clean(name):
    """A name fit to offer: no domain, no disambiguating parentheses."""
    if re.search(r"\.(com|net|org|co|jp|kr|io)\b", name, re.I) or re.search(r"[(（]", name):
        return False
    return True


def default_script(en, country):
    """docs/moonshot.md §5.4: "... Korea" Korean, "... Japan" English; else by the country."""
    if re.search(r"\bKorea$", en):
        return "ko"
    if re.search(r"\bJapan$", en):
        return "en"
    return {"KR": "ko", "JP": "ja"}.get(country, "en")


def country_of(e):
    """A brand of several countries: Korea, then Japan, else the first."""
    countries = e["countries"]
    return "KR" if "KR" in countries else "JP" if "JP" in countries else (sorted(countries)[0] if countries else "")


def bare_names(e):
    """The names a brand is searched by alone, by language: its labels without their legal form."""
    return {lang: [plain(e[lang])] if e[lang] else [] for lang in ("en", "ko", "ja")}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--min-sitelinks", type=int, default=8)
    args = ap.parse_args()
    items = {}
    queries = [(f"{cls}-{args.min_sitelinks}", QUERY % {"cls": cls, "min": args.min_sitelinks}, name, True) for cls, name in CLASSES.items()]
    queries += [(f"industry-{lo}-{hi}", INDUSTRY % {"lo": max(lo, args.min_sitelinks), "hi": hi}, f"industry {lo}-{hi}", False) for lo, hi in SLICES]
    for key, query, name, by_class in queries:
        rows = fetch(key, query)
        print(f"{name:26} {len(rows):6} rows", file=sys.stderr)
        for r in rows:
            v = lambda k: r.get(k, {}).get("value", "")  # noqa: E731
            item = v("item")
            entry = items.setdefault(item, {"links": int(v("links") or 0), "en": v("en"), "ko": v("ko"), "ja": v("ja"),
                                            "kana": set(), "countries": set(), "alts": set(), "by_class": False})
            entry["by_class"] = entry["by_class"] or by_class
            for lang in ("en", "ko", "ja"):
                if not entry[lang] and v(lang):
                    entry[lang] = v(lang)
            if v("kana"):
                entry["kana"].add(v("kana"))
            if v("country"):
                entry["countries"].add(v("country"))
            for lang in ("ko", "ja", "en"):
                for alt in v(lang + "Alts").split("|"):
                    if alt:
                        entry["alts"].add((lang, alt))
    admitted = admission.admit(CACHE, items, country_of, bare_names)
    print(f"{len(admitted)} of {len(items)} candidates admitted (corpora/brands/admission.tsv)", file=sys.stderr)
    lines = []
    for item, e in sorted(items.items(), key=lambda kv: -kv[1]["links"]):
        if item.rsplit("/", 1)[-1] not in admitted:
            continue
        country = country_of(e)
        script = default_script(e["en"], country)
        if script != "en" and not e[script]:
            script = "en"
        # Every name in the default script, without its legal form: the engine offers the one
        # closest in length to what was typed (도요타 → トヨタ, not トヨタ自動車).
        names = []
        for s in [e[script], *(a for k, a in sorted(e["alts"]) if k == script)]:
            n = plain(s)
            if n and writes(n, script) and clean(n) and "\t" not in n and "|" not in n and n not in names and len(n) <= 40:
                names.append(n)
        keys = set()
        for s in [e["ko"], e["ja"], *e["kana"], *(a for k, a in e["alts"] if k in ("ko", "ja"))]:
            n = norm_cjk(plain(s)) if s else ""
            for k in (n, CORPORATE.sub("", n)):
                if len(k) >= 2:
                    keys.add(k)
        for s in [e["en"], *(a for k, a in e["alts"] if k == "en")]:
            n = norm_en(plain(s)) if s else ""
            for k in (n, CORPORATE.sub("", n)):
                if len(k) >= 3:
                    keys.add(k)
        if not names or not keys:
            continue
        lines.append("|".join(names) + "\t" + "\t".join(sorted(keys)))
    with open(OUT, "w", encoding="utf-8") as f:
        f.write("# Brand names (docs/moonshot.md §5.4), from Wikidata (CC0), by build_brands.py.\n")
        f.write("# The names in the default script (|-separated), then the normalized spellings it is typed as.\n")
        f.write("\n".join(lines) + "\n")
    print(f"{len(lines)} brands -> {OUT} ({os.path.getsize(OUT)} bytes)", file=sys.stderr)


if __name__ == "__main__":
    main()
