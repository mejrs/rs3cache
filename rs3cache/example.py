from ffi.rs3cache_py import *

mapsquares = MapSquares()

lumbridge = mapsquares.get(50,50)
print(lumbridge.metadata)
for _, loc in zip(range(10), lumbridge.locations()):
    print(loc)

defs = Definitions()
print(defs.location_configs[89602])

assert "Fire" in defs.location_configs[6].actions
assert defs.location_configs[1].name == "Crate"
assert defs.location_configs[118445].params.get(8178) == 50923

print(defs.npc_configs[0])
assert defs.npc_configs[0].name == "Hans"
