from rs3 import *

mapsquares = MapSquares(path = "../test_data/rs3_cache")

lumbridge = mapsquares.get(50,50)

for _, loc in zip(range(10), lumbridge.locations()):
    print(loc)

location_configs = get_location_configs(path = "../test_data/rs3_cache")

print(location_configs[89602])

assert "Fire" in location_configs[6].actions
assert location_configs[1].name == "Crate"
assert location_configs[118445].params.get(8178) == 50923

npc_configs = get_npc_configs(path = "../test_data/rs3_cache")

print(npc_configs[0])
assert npc_configs[0].name == "Hans"


def test():
    pass