#!/bin/env python3

import argparse
import pathlib
import collections
import json
import shutil

def main():
    parser = argparse.ArgumentParser(
        description='Generate JSON index for CHIP8 roms')
    parser.add_argument('-o',
                        '--output',
                        metavar='FILE.json',
                        required=True,
                        help='Output index path')
    parser.add_argument('-v',
                        '--verbose',
                        action='store_true',
                        help='More verbose output')

    parser.add_argument('root', help='Source directory searched for .ch8 ROMs')
    args = parser.parse_args()

    root = pathlib.Path(args.root)
    dst = pathlib.Path(args.output)
    dst_dir = dst.parent

    obj = collections.defaultdict(dict)

    subdirs = {'revival-pack/games': {'source': 'https://github.com/dmatlack/chip8'}}

    rom_id = 0
    for d, attrs in subdirs.items():
        p = root / d
        for f in p.glob('*.ch8'):
            rom_id += 1
            rom_name = f.with_suffix('').name.split()[0].lower()
            rom_name = f'{rom_id:03}-{rom_name}'
            obj[rom_name] = {
                'name': f.with_suffix('').name}
            obj[rom_name].update(attrs)
            shutil.copy(f, dst_dir / rom_name)

    if args.verbose:
        print(json.dumps(obj, sort_keys=True, indent=4))
    with open(dst, 'w') as f:
        json.dump(obj, f)


if __name__ == "__main__":
    main()
