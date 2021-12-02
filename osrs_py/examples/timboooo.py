from osrs import *
from itertools import product

"""
An utility for tracking when trees are added or removed.
"""

new = MapSquares()

#path to any old cache
old = MapSquares(path = "C:/Users/bruno/runescape/cache/raw")

loc_config = get_location_configs()

for i, j in product(range(100), range(200)):
    try:
        new_objs = set(new.get(i,j).locations())
    except (ValueError, FileNotFoundError, RuntimeError):
        new_objs = set()

    try:
        old_objs = set(old.get(i,j).locations())
    except (ValueError, FileNotFoundError, RuntimeError):
        old_objs = set()

    added = new_objs - old_objs
    removed = old_objs - new_objs

    if added:
        for loc in added:
            pos = (loc.plane, loc.i << 6 | loc.x,  loc.j << 6 | loc.y)
            name = loc_config[loc.id].name
            if name and ("tree" in name or "Tree" in name):
                print("added", pos, loc.id, name)

    if removed:
        for loc in removed:
            pos = (loc.plane, loc.i << 6 | loc.x,  loc.j << 6 | loc.y)
            name = loc_config[loc.id].name
            if name and ("tree" in name or "Tree" in name):
                print("removed", pos, loc.id, name)
