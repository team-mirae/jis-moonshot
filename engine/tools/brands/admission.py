"""Which Wikidata candidates are brands worth suggesting (docs/moonshot.md §5.4), for
build_brands.py. A candidate is admitted when it is a company or a brand (not a person, a place, a
school, a club, a party or a government body: NOT_BRANDS) and either

1. **it is in a major stock index**, or big enough to pass that bar easily: a constituent of the
   S&P 500, NASDAQ-100, Dow Jones, KOSPI 200, Nikkei 225, FTSE 100, DAX, CAC 40, EURO STOXX 50,
   Swiss Market Index or Hang Seng (Wikidata's "part of", and the Components sections of English
   Wikipedia where Wikidata has too few), or with a market capitalization or revenue over its
   country's bar (BARS); or
2. **its name, searched alone, is the brand**: Wikipedia's article at the bare name (in English,
   Korean or Japanese) is the brand's or redirects to it, or gets fewer readers than the brand's;
   and the brand's article is widely read (FLOOR views in 60 days). A disambiguation page at the
   bare name asks four times that.

Dissolved companies (nothing official left to find) and classes or concepts (television) are left
out either way.

Everything fetched is cached in corpora/brands/ (gitignored): facts.json, constituents.json and
views-*.json. Delete them to fetch again.
"""

import json
import os
import sys
import time
import urllib.parse
import urllib.request

AGENT = "jis-moonshot-brands/0.2 (https://github.com/team-mirae/jis-moonshot)"
SPARQL = "https://query.wikidata.org/sparql"

# Index items whose constituents Wikidata records ("part of" P361, or the index's "has part").
INDICES = {
    "Q242345": "S&P 500",
    "Q507306": "NASDAQ-100",
    "Q180816": "Dow Jones",
    "Q466496": "FTSE 100",
    "Q155718": "DAX",
    "Q648828": "CAC 40",
    "Q981010": "EURO STOXX 50",
    "Q666714": "SMI",
    "Q507338": "Nikkei 225",
    "Q12620592": "KOSPI 200",
    "Q424846": "Hang Seng",
}
# Where Wikidata has few or none: the links in the Components sections of English Wikipedia.
COMPONENT_PAGES = {"Nikkei 225": "Nikkei 225", "KOSPI 200": "KOSPI 200", "Hang Seng": "Hang Seng Index"}

# "Easily passes the bar" of its country's main index, in US dollars: (market capitalization,
# revenue). The S&P 500's entry is about $20 billion of market value; KOSPI 200's and Nikkei 225's
# are far lower.
BARS = {"US": (20e9, 10e9), "KR": (2e9, 3e9), "JP": (7e9, 7e9)}
OTHER_BAR = (20e9, 10e9)

# Approximate US dollars per unit (Wikidata currency items).
USD = {
    "Q4917": 1.0, "Q4916": 1.08, "Q8146": 0.0067, "Q202040": 0.00072, "Q25224": 1.27,
    "Q39099": 0.14, "Q25344": 1.12, "Q31015": 0.128, "Q208526": 0.031, "Q80524": 0.012,
    "Q1104069": 0.73, "Q259502": 0.66, "Q122922": 0.095, "Q25417": 0.145, "Q132643": 0.093,
    "Q173117": 0.18, "Q41044": 0.011, "Q190951": 0.74,
}

# Classes that are never a brand: people and characters.
NOT_ORGANIZATIONS = {"Q5", "Q15632617", "Q95074", "Q15773347", "Q4271324"}

# Organizations that aren't brands (the owner's bar is companies and their brands): schools, sports
# clubs, religions, parties, governments and their agencies, nonprofits, places. Left out unless
# also classed as a company, business or brand (COMMERCIAL).
NOT_BRANDS = {
    # schools and universities
    "Q23002054", "Q902104", "Q15936437", "Q3918", "Q1188663", "Q1377182", "Q557206", "Q3660535", "Q1371037",
    "Q615150", "Q6540832", "Q189004", "Q2385804", "Q3914", "Q845392", "Q875538", "Q21032622", "Q6040928", "Q7638697",
    # sports clubs, teams, leagues and governing bodies
    "Q476028", "Q847017", "Q103229495", "Q4438121", "Q623109", "Q15991290", "Q15991303", "Q20639856", "Q10651067",
    "Q11422536", "Q2485448", "Q12973014",
    # religion
    "Q1530022", "Q477544", "Q13414953", "Q879146", "Q11038979",
    # politics, government, international bodies, movements
    "Q7278", "Q2023214", "Q7210356", "Q3778417", "Q2257409", "Q6738353", "Q327333", "Q1752939", "Q110376455",
    "Q98675100", "Q17505024", "Q15627509", "Q732717", "Q11204", "Q1752346", "Q375928", "Q66344", "Q484652",
    "Q245065", "Q15925165", "Q139312011", "Q1345691", "Q5266746", "Q17127659", "Q6762254", "Q138796", "Q2738074",
    "Q2659904", "Q7188",
    # nonprofits, charities, think tanks, research institutes
    "Q431603", "Q155271", "Q708676", "Q163740", "Q31855", "Q136410232", "Q120442195", "Q48204", "Q15911314",
    # places, buildings, museums
    "Q755707", "Q719487", "Q15284", "Q515", "Q486972", "Q3957", "Q1549591", "Q5119", "Q1311958", "Q1317590",
    "Q26263224", "Q33506", "Q207694", "Q16917",
    # not organizations at all
    "Q1047113", "Q11862829", "Q482980", "Q2927074", "Q24856", "Q1255441", "Q91615393", "Q1700154",
}
# Never a brand, whatever else they're classed as: central banks (the Bank of England is also a
# "business"), islands, mines, facilities.
NEVER_BRANDS = {"Q66344", "Q23442", "Q820477", "Q13226383"}
COMMERCIAL = {"Q4830453", "Q891723", "Q6881511", "Q783794", "Q431289", "Q167037", "Q1589009", "Q658255", "Q161726",
              "Q219577", "Q778575", "Q21980538", "Q5621421", "Q134161", "Q167270", "Q2519914"}

# Readers in 60 days a well-known brand's article has, by wiki.
FLOOR = {"en": 30000, "ja": 12000, "ko": 3000}
DISAMBIGUATION_FACTOR = 4


def request(url, data=None, accept="application/json"):
    for attempt in range(5):
        try:
            req = urllib.request.Request(url, data=data, headers={"User-Agent": AGENT, "Accept": accept})
            with urllib.request.urlopen(req, timeout=120) as r:
                return json.load(r)
        except Exception as e:  # noqa: BLE001 - a timeout or a rate limit: wait and try again
            print(f"  {e}; retrying", file=sys.stderr)
            time.sleep(5 * (attempt + 1))
    raise RuntimeError(f"giving up on {url[:120]}")


def sparql(query):
    data = urllib.parse.urlencode({"query": query, "format": "json"}).encode()
    return request(SPARQL, data, "application/sparql-results+json")["results"]["bindings"]


def cached(cache, name, build):
    path = os.path.join(cache, name)
    if os.path.exists(path):
        with open(path, encoding="utf-8") as f:
            return json.load(f)
    value = build()
    with open(path, "w", encoding="utf-8") as f:
        json.dump(value, f, ensure_ascii=False)
    return value


FACTS = """
SELECT ?item (GROUP_CONCAT(DISTINCT STRAFTER(STR(?class), "entity/"); separator=" ") AS ?classes)
       (SAMPLE(?evidence) AS ?org) ?enTitle ?koTitle ?jaTitle WHERE {
  VALUES ?item { %s }
  OPTIONAL { ?item wdt:P31 ?class }
  OPTIONAL { ?item wdt:P414|wdt:P159|wdt:P1454|wdt:P749|wdt:P2139|wdt:P2226|wdt:P1128|wdt:P112|wdt:P169|wdt:P1056 ?evidence }
  OPTIONAL { ?en schema:about ?item ; schema:isPartOf <https://en.wikipedia.org/> ; schema:name ?enTitle }
  OPTIONAL { ?ko schema:about ?item ; schema:isPartOf <https://ko.wikipedia.org/> ; schema:name ?koTitle }
  OPTIONAL { ?ja schema:about ?item ; schema:isPartOf <https://ja.wikipedia.org/> ; schema:name ?jaTitle }
}
GROUP BY ?item ?enTitle ?koTitle ?jaTitle
"""

MONEY = """
SELECT ?item ?prop ?amount ?unit ?time WHERE {
  VALUES ?item { %s }
  VALUES ?prop { p:P2226 p:P2139 }
  ?item ?prop ?st .
  ?st ?psv ?v . ?v wikibase:quantityAmount ?amount ; wikibase:quantityUnit ?unit .
  FILTER(STRSTARTS(STR(?psv), "http://www.wikidata.org/prop/statement/value/"))
  OPTIONAL { ?st pq:P585 ?time }
}
"""


def facts(cache, ids):
    """Per item: its classes, whether it shows itself an organization, its wiki titles, and its
    latest market capitalization and revenue in dollars."""

    def build():
        out = {}
        ids_sorted = sorted(ids)
        for start in range(0, len(ids_sorted), 250):
            batch = ids_sorted[start:start + 250]
            values = " ".join("wd:" + i for i in batch)
            for r in sparql(FACTS % values):
                v = lambda k: r.get(k, {}).get("value", "")  # noqa: E731
                q = v("item").rsplit("/", 1)[-1]
                e = out.setdefault(q, {"classes": [], "org": False, "titles": {}, "money": {}})
                e["classes"] = sorted(set(e["classes"]) | set(v("classes").split()))
                e["org"] = e["org"] or bool(v("org"))
                for lang in ("en", "ko", "ja"):
                    if v(lang + "Title"):
                        e["titles"][lang] = v(lang + "Title")
            latest = {}
            for r in sparql(MONEY % values):
                q = r["item"]["value"].rsplit("/", 1)[-1]
                prop = "cap" if r["prop"]["value"].endswith("P2226") else "revenue"
                rate = USD.get(r["unit"]["value"].rsplit("/", 1)[-1])
                if rate is None:
                    continue
                when = r.get("time", {}).get("value", "")
                dollars = abs(float(r["amount"]["value"])) * rate
                key = (q, prop)
                # The latest figure; undated ones only where nothing is dated.
                if key not in latest or (when, dollars) > latest[key]:
                    latest[key] = (when, dollars)
            for (q, prop), (_, dollars) in latest.items():
                out.setdefault(q, {"classes": [], "org": False, "titles": {}, "money": {}})["money"][prop] = dollars
            print(f"  facts {start + len(batch)}/{len(ids_sorted)}", file=sys.stderr)
        return out

    return cached(cache, "facts.json", build)


LIFESPAN = """
SELECT ?item (SAMPLE(BOUND(?dissolved)) AS ?gone) (SAMPLE(BOUND(?super)) AS ?concept) (MAX(?when) AS ?latest) WHERE {
  VALUES ?item { %s }
  OPTIONAL { ?item wdt:P576 ?dissolved }
  OPTIONAL { ?item wdt:P279 ?super }
  OPTIONAL { ?item p:P2139|p:P2226 ?figure . ?figure pq:P585 ?when }
}
GROUP BY ?item
"""


def lifespans(cache, ids):
    """Item → [dissolved, has "subclass of", the year of its latest revenue or market value], for
    `ids`."""
    known = {}
    path = os.path.join(cache, "lifespans.json")
    if os.path.exists(path):
        with open(path, encoding="utf-8") as f:
            known = json.load(f)
    todo = sorted(set(ids) - set(known))
    for start in range(0, len(todo), 250):
        batch = todo[start:start + 250]
        for q in batch:
            known[q] = [False, False, 0]
        for r in sparql(LIFESPAN % " ".join("wd:" + q for q in batch)):
            q = r["item"]["value"].rsplit("/", 1)[-1]
            known[q][0] |= r.get("gone", {}).get("value") == "true"
            known[q][1] |= r.get("concept", {}).get("value") == "true"
            latest = r.get("latest", {}).get("value", "")
            known[q][2] = max(known[q][2], int(latest[:4]) if latest[:4].isdigit() else 0)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(known, f)
    return known


def constituents(cache):
    """Item → the indices it is (or was) in."""

    def build():
        out = {}
        values = " ".join("wd:" + q for q in INDICES)
        query = f"""SELECT DISTINCT ?item ?index WHERE {{ VALUES ?index {{ {values} }}
          {{ ?item wdt:P361 ?index }} UNION {{ ?item p:P361/ps:P361 ?index }} UNION {{ ?index wdt:P527 ?item }} }}"""
        for r in sparql(query):
            q = r["item"]["value"].rsplit("/", 1)[-1]
            out.setdefault(q, []).append(INDICES[r["index"]["value"].rsplit("/", 1)[-1]])
        for index, page in COMPONENT_PAGES.items():
            api = "https://en.wikipedia.org/w/api.php?"
            sections = request(api + urllib.parse.urlencode({"action": "parse", "page": page, "prop": "sections", "format": "json", "redirects": 1}))
            numbers = []
            inside = None
            for s in sections["parse"]["sections"]:
                level = int(s["toclevel"])
                if s["line"].strip().lower() in ("components", "constituents", "constituent stocks"):
                    inside = level
                    numbers.append(s["index"])
                elif inside is not None and level > inside:
                    numbers.append(s["index"])
                elif inside is not None:
                    inside = None
            titles = set()
            for n in numbers:
                parsed = request(api + urllib.parse.urlencode({"action": "parse", "page": page, "prop": "links", "section": n, "format": "json", "redirects": 1}))
                titles.update(l["*"] for l in parsed["parse"]["links"] if l.get("ns") == 0 and "exists" in l)
            found = 0
            titles = sorted(titles)
            for start in range(0, len(titles), 50):
                batch = titles[start:start + 50]
                data = request(api + urllib.parse.urlencode({"action": "query", "titles": "|".join(batch), "prop": "pageprops",
                                                             "ppprop": "wikibase_item", "redirects": 1, "format": "json", "formatversion": 2}))
                for p in data["query"].get("pages", []):
                    q = p.get("pageprops", {}).get("wikibase_item")
                    if q:
                        out.setdefault(q, [])
                        if index not in out[q]:
                            out[q].append(index)
                            found += 1
            print(f"  {index}: {found} from {page}", file=sys.stderr)
        return out

    return cached(cache, "constituents.json", build)


def views(cache, lang, titles):
    """Title → (the page it resolves to, readers in 60 days, whether it's a disambiguation page);
    a title with no page is left out."""
    path = f"views-{lang}.json"
    known = {}
    if os.path.exists(os.path.join(cache, path)):
        with open(os.path.join(cache, path), encoding="utf-8") as f:
            known = json.load(f)
    todo = sorted(t for t in set(titles) if t and t not in known)
    api = f"https://{lang}.wikipedia.org/w/api.php?"
    for start in range(0, len(todo), 50):
        batch = todo[start:start + 50]
        params = {"action": "query", "titles": "|".join(batch), "prop": "pageviews|pageprops", "ppprop": "disambiguation",
                  "pvipdays": 60, "redirects": 1, "format": "json", "formatversion": 2}
        # The API gives the views of a few pages at a time: follow its continuation, merging.
        pages, moves_of = {}, {"normalized": {}, "redirects": {}}
        cont = {}
        while True:
            reply = request(api + urllib.parse.urlencode({**params, **cont}))
            data = reply.get("query", {})
            for step in moves_of:
                moves_of[step].update({m["from"]: m["to"] for m in data.get(step, [])})
            for p in data.get("pages", []):
                old = pages.setdefault(p["title"], p)
                if p.get("pageviews"):
                    old["pageviews"] = {**(old.get("pageviews") or {}), **{k: v for k, v in p["pageviews"].items() if v is not None}}
            if "continue" not in reply:
                break
            cont = reply["continue"]
        target = {t: t for t in batch}
        for step in ("normalized", "redirects"):
            target = {t: moves_of[step].get(to, to) for t, to in target.items()}
        for t in batch:
            p = pages.get(target[t])
            if not p or p.get("missing"):
                known[t] = None
                continue
            seen = sum(v or 0 for v in (p.get("pageviews") or {}).values())
            known[t] = [p["title"], seen, "disambiguation" in p.get("pageprops", {})]
        if start // 50 % 20 == 0:
            print(f"  views {lang} {start + len(batch)}/{len(todo)}", file=sys.stderr)
            with open(os.path.join(cache, path), "w", encoding="utf-8") as f:
                json.dump(known, f, ensure_ascii=False)
        time.sleep(0.1)
    with open(os.path.join(cache, path), "w", encoding="utf-8") as f:
        json.dump(known, f, ensure_ascii=False)
    return known


def admit(cache, items, country_of, names_of):
    """{item id: reason} for the candidates admitted (see the module docstring). `items` maps
    Wikidata URIs to build_brands' entries; `country_of(entry)` gives its country code and
    `names_of(entry)` its bare names by language ({"en": [...], "ko": [...], "ja": [...]})."""
    ids = {uri.rsplit("/", 1)[-1]: e for uri, e in items.items()}
    fact = facts(cache, ids)
    member = constituents(cache)
    # Every brand article and every bare name, by wiki.
    wanted = {"en": set(), "ko": set(), "ja": set()}
    for q, e in ids.items():
        f = fact.get(q, {})
        for lang, title in f.get("titles", {}).items():
            wanted[lang].add(title)
        for lang, names in names_of(e).items():
            wanted[lang].update(names)
    seen = {lang: views(cache, lang, titles) for lang, titles in wanted.items()}

    admitted, rejected = {}, {}
    for q, e in ids.items():
        f = fact.get(q, {"classes": [], "org": False, "titles": {}, "money": {}})
        classes = set(f["classes"])
        if classes & NOT_ORGANIZATIONS:
            rejected[q] = "a person or character"
            continue
        if classes & NEVER_BRANDS or (classes & NOT_BRANDS and not classes & COMMERCIAL):
            rejected[q] = "an organization but not a brand"
            continue
        # Found by a company or brand class, or with something only an organization has.
        if not (e.get("by_class") or f["org"]):
            rejected[q] = "not an organization"
            continue
        if q in member:
            admitted[q] = "index: " + ", ".join(sorted(member[q]))
            continue
        cap_bar, revenue_bar = BARS.get(country_of(e), OTHER_BAR)
        money = f["money"]
        if money.get("cap", 0) >= cap_bar or money.get("revenue", 0) >= revenue_bar:
            admitted[q] = "size: " + ", ".join(f"{k} ${v / 1e9:.0f}B" for k, v in sorted(money.items()))
            continue
        reason = None
        for lang, title in sorted(f["titles"].items()):
            page = seen[lang].get(title)
            if not page or page[1] < FLOOR[lang]:
                continue
            for name in names_of(e).get(lang, []):
                bare = seen[lang].get(name)
                if bare is None:
                    continue
                if bare[0] == page[0]:
                    reason = f"name: {lang} {name} is the brand's page"
                elif bare[2]:
                    if page[1] >= DISAMBIGUATION_FACTOR * FLOOR[lang]:
                        reason = f"name: {lang} {name} ambiguous, brand read {page[1]}"
                elif page[1] >= bare[1]:
                    reason = f"name: {lang} {name} read less ({bare[1]}) than the brand ({page[1]})"
                if reason:
                    break
            if reason:
                break
        if reason:
            admitted[q] = reason
        else:
            rejected[q] = "neither in an index nor known by its name alone"
    # Gone (dissolved: nothing official to find, unless it reports figures from 2020 on, as
    # Telefónica does under a wrong date) or not a thing at all (a class with a lowercase name:
    # television; Lego has "subclass of" too).
    life = lifespans(cache, admitted)
    for q in list(admitted):
        dissolved, subclass, latest = life.get(q, [False, False, 0])
        # In a current constituent list (Wikipedia's, not Wikidata's, which keeps former members):
        # alive, whatever Wikidata says (Woori Financial Group).
        listed = any(index in member.get(q, []) for index in COMPONENT_PAGES)
        gone = dissolved and latest < 2020 and not listed
        concept = subclass and not any(c.isupper() for c in ids[q]["en"])
        if gone or concept:
            rejected[q] = "dissolved" if gone else "a class or concept"
            del admitted[q]
    with open(os.path.join(cache, "admission.tsv"), "w", encoding="utf-8") as f:
        for q in sorted(ids, key=lambda q: -ids[q]["links"]):
            e = ids[q]
            verdict = ("+ " + admitted[q]) if q in admitted else ("- " + rejected.get(q, ""))
            f.write(f"{q}\t{e['en']}\t{e['ko']}\t{e['ja']}\t{verdict}\n")
    return admitted

