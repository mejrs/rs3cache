from rs3 import Index, CacheNotFoundError

def test_not_found():
    try:
        Index(12, path = "blah")
    except CacheNotFoundError as e:
        assert "js5-61.JCACHE" in str(e), e
    else:
        raise RuntimeError("this should have failed")