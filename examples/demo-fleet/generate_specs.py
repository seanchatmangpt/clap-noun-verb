import itertools, os, yaml

out_dir = "/tmp/demo-fleet-specs"
os.makedirs(out_dir, exist_ok=True)

noun_counts = [1, 2]
verb_counts = [1, 2, 3]
arg_styles = ["none", "one", "two"]

# Cartesian product: 2 x 3 x 3 = 18; add 2 more distinct variants for 20 total.
combos = list(itertools.product(noun_counts, verb_counts, arg_styles))
# extra distinct variants: 3 nouns/1 verb/none, 3 nouns/2 verbs/one
combos += [(3, 1, "none"), (3, 2, "one")]
assert len(combos) == 20, len(combos)

nouns_pool = ["fleet", "storage", "network", "compute"]

def make_arg(style):
    if style == "none":
        return []
    if style == "one":
        return [{"name": "target", "arg_type": "String", "doc": "Target identifier", "required": True}]
    return [
        {"name": "target", "arg_type": "String", "doc": "Target identifier", "required": True},
        {"name": "verbose", "arg_type": "bool", "doc": "Verbose output", "required": False, "is_flag": True},
    ]

specs = []
for i, (noun_count, verb_count, arg_style) in enumerate(combos):
    name = f"demo-fleet-{i:02d}"
    verbs = []
    for n in range(noun_count):
        noun = nouns_pool[n % len(nouns_pool)]
        for v in range(verb_count):
            verb_name = ["status", "sync", "report"][v % 3]
            verbs.append({
                "name": verb_name,
                "noun": noun,
                "doc": f"{verb_name} for {noun} (variant {i})",
                "args": make_arg(arg_style),
            })
    spec = {
        "name": name,
        "about": f"Demo fleet CLI variant {i}: {noun_count} noun(s) x {verb_count} verb(s), {arg_style} args",
        "version": "0.1.0",
        "author": "clap-noun-verb demo fleet",
        "verbs": verbs,
    }
    path = os.path.join(out_dir, f"{name}.yaml")
    with open(path, "w") as f:
        yaml.safe_dump(spec, f, sort_keys=False)
    specs.append((name, noun_count, verb_count, arg_style, len(verbs)))

for s in specs:
    print(s)
