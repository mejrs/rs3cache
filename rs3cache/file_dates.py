from rs3cache import *
from datetime import datetime

# Iterate over all the metadata of this index (47 is models)
for id, meta in Index(47).metadatas():
    # The version field contains an utc timestamp
    #
    # Some may still use the non date format,
    # in which case this will output something around the unix epoch.
    time = datetime.utcfromtimestamp(meta.version).strftime('%Y-%m-%d %H:%M:%S')

    # We just print here, you should do something more imaginative :)
    print(id, time)
