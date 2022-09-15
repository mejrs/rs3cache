from osrs import *

# Iterate over all the metadata of this index (12 is client side scripts)
for id, meta in Index(12, path = "../test_data/osrs_cache").metadatas():

    # We just print here, you should do something more imaginative :)
    print(id, meta.version)

def test():
    pass