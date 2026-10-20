#!/usr/bin/env python3
"""Summarize tks-sim --natural-builds report without misstating acquisition.

Only Python standard library. Each sample is one paired normal-starter seed,
not an independent natural human win-rate sample or a full boss matchup study.
"""
import json
import sys
from collections import defaultdict
from pathlib import Path


def summarize(payload):
    if payload.get('schema_version') != 1:
        raise ValueError('unsupported report schema')
    cells = payload.get('cells')
    if not isinstance(cells, list) or not cells:
        raise ValueError('no natural-build results')
    sums = defaultdict(lambda: defaultdict(int))
    seen = set()
    metrics = ('samples', 'baseline_wins', 'focused_wins', 'complete_core',
               'complete_support', 'core_distinct_sum', 'support_distinct_sum',
               'reward_screens', 'focus_offer_screens', 'focus_picks')
    for cell in cells:
        ident = (cell['id'], cell['ascension'])
        if ident in seen or not isinstance(ident[1], int) or not 0 <= ident[1] <= 10:
            raise ValueError(f'duplicate or malformed build cell: {ident}')
        seen.add(ident)
        n = cell['samples']
        if not isinstance(n, int) or n <= 0:
            raise ValueError(f'invalid denominator: {ident}')
        if any(not isinstance(cell[key], int) or cell[key] < 0 for key in metrics):
            raise ValueError(f'negative/non-integer metric: {ident}')
        if any(cell[key] > n for key in
               ('baseline_wins', 'focused_wins', 'complete_core', 'complete_support')):
            raise ValueError(f'outcome exceeds runs: {ident}')
        if not 0 <= cell['focus_picks'] <= cell['focus_offer_screens'] <= cell['reward_screens']:
            raise ValueError(f'inconsistent acquisition: {ident}')
        if not 0 <= cell['core_distinct_sum'] <= 4*n:
            raise ValueError(f'invalid four-core count: {ident}')
        if not 0 <= cell['support_distinct_sum'] <= 2*n:
            raise ValueError(f'invalid two-support count: {ident}')
        for key in metrics:
            sums[(cell['hero'], cell['id'])][key] += cell[key]
    return sums, len(seen)


def main(argv):
    if len(argv) != 2:
        print('usage: python3 scripts/analyze_natural_builds.py tks-natural-builds.json',
              file=sys.stderr)
        return 2
    data=json.loads(Path(argv[1]).read_text(encoding='utf-8'))
    rows,cells=summarize(data)
    print(f'Builds: {len(rows)}; cells: {cells}')
    print('Hero | Build | Base wins | Focus wins | Full 4-core | Full 2-support | Offered/picked')
    print('--- | --- | ---: | ---: | ---: | ---: | ---:')
    for (hero,build),v in sorted(rows.items()):
        n=v['samples']
        print(f"{hero} | {build} | {v['baseline_wins']}/{n} | {v['focused_wins']}/{n} | "
              f"{v['complete_core']}/{n} | {v['complete_support']}/{n} | "
              f"{v['focus_offer_screens']}/{v['focus_picks']}")
    print('Caution: single-seed ascension cells are deterministic smoke, not'
          ' mature build balance or independent boss counters.')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv))
