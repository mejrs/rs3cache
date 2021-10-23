from osrsimport *

mapsquares = MapSquares()

lumbridge = mapsquares.get(50,50)
print(lumbridge.metadata)
for _, loc in zip(range(10), lumbridge.locations()):
    print(loc)

location_configs = get_location_configs()

print(location_configs[89602])

assert "Fire" in location_configs[6].actions
assert location_configs[1].name == "Crate"
assert location_configs[118445].params.get(8178) == 50923

npc_configs = get_npc_configs()

print(npc_configs[0])
assert npc_configs[0].name == "Hans"
