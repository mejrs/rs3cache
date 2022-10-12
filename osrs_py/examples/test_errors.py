from osrs import Index, CacheNotFoundError

def test_not_found():
    try:
        Index(12, path = "blah")
    except CacheNotFoundError as e:
        assert "xteas.json OR keys.json" in str(e)
    else:
        raise RuntimeError("this should have failed")