from osrs import *

mapsquares = MapSquares(path = "../test_data/osrs_cache")

lumbridge = mapsquares.get(50,50)

for _, loc in zip(range(10), lumbridge.locations()):
    print(loc)

location_configs = get_location_configs(path = "../test_data/osrs_cache")

print(location_configs[6560])

assert "Fire" in location_configs[6].actions
assert location_configs[1].name == "Crate"

def test():
    pass