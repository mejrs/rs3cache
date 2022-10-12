# This locates all locations (a.k.a. objects) that are named "[ph]".

# Imports the library. You must have built python wheels to make this work.
from rs3 import MapSquares, get_location_configs, FileMissingError

# Load all location properties (e.g. their name, models and so on).
loc_configs = get_location_configs(path = "../test_data/rs3_cache")

# MapSquares implements the iterator protocol,
# so we can do for .. in .. to traverse all mapsquares.
# We could also use its .get(i,j) method
# to get the mapsquare at position i, j.
for mapsquare in MapSquares(path = "../test_data/rs3_cache"):
	try:
		locations = mapsquare.locations()
	except FileMissingError:
		# not all mapsquares contain locations.
		pass
	else:
		for loc in locations:
			if loc.id == 106208:
				print("hi")

def test():
	pass