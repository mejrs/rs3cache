(function() {var implementors = {
"rs3cache_backend":[["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/decoder/enum.DecodeError.html\" title=\"enum rs3cache_backend::decoder::DecodeError\">DecodeError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/decoder/struct.Unimplemented.html\" title=\"struct rs3cache_backend::decoder::Unimplemented\">Unimplemented</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/buf/enum.ReadError.html\" title=\"enum rs3cache_backend::buf::ReadError\">ReadError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/buf/struct.Eof.html\" title=\"struct rs3cache_backend::buf::Eof\">Eof</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, <a class=\"struct\" href=\"https://docs.rs/serde_json/1.0.79/serde_json/error/struct.Error.html\" title=\"struct serde_json::error::Error\">Error</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/error/enum.CacheError.html\" title=\"enum rs3cache_backend::error::CacheError\">CacheError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/error/struct.XteaLoad.html\" title=\"struct rs3cache_backend::error::XteaLoad\">XteaLoad</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.Blank.html\" title=\"struct rs3cache_backend::index::Blank\">Blank</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.Crc.html\" title=\"struct rs3cache_backend::index::Crc\">Crc</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/buf/enum.ReadError.html\" title=\"enum rs3cache_backend::buf::ReadError\">ReadError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/buf/struct.NotExhausted.html\" title=\"struct rs3cache_backend::buf::NotExhausted\">NotExhausted</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.Blank.html\" title=\"struct rs3cache_backend::index::Blank\">Blank</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/buf/enum.ReadError.html\" title=\"enum rs3cache_backend::buf::ReadError\">ReadError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/buf/struct.NotNulTerminated.html\" title=\"struct rs3cache_backend::buf::NotNulTerminated\">NotNulTerminated</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, DecoderError&gt;, <a class=\"enum\" href=\"rs3cache_backend/decoder/enum.DecodeError.html\" title=\"enum rs3cache_backend::decoder::DecodeError\">DecodeError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/decoder/struct.BZip2.html\" title=\"struct rs3cache_backend::decoder::BZip2\">BZip2</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/io/error/struct.Error.html\" title=\"struct std::io::error::Error\">Error</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/decoder/enum.DecodeError.html\" title=\"enum rs3cache_backend::decoder::DecodeError\">DecodeError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/decoder/struct.Zlib.html\" title=\"struct rs3cache_backend::decoder::Zlib\">Zlib</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.FileMissing.html\" title=\"struct rs3cache_backend::index::FileMissing\">FileMissing</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, Error&gt;, <a class=\"enum\" href=\"rs3cache_backend/error/enum.CacheError.html\" title=\"enum rs3cache_backend::error::CacheError\">CacheError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/error/struct.CannotOpen.html\" title=\"struct rs3cache_backend::error::CannotOpen\">CannotOpen</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, Error&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.Corrupted.html\" title=\"struct rs3cache_backend::index::Corrupted\">Corrupted</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.ArchiveMissingNamed.html\" title=\"struct rs3cache_backend::index::ArchiveMissingNamed\">ArchiveMissingNamed</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/error/enum.CacheError.html\" title=\"enum rs3cache_backend::error::CacheError\">CacheError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/error/struct.Xtea.html\" title=\"struct rs3cache_backend::error::Xtea\">Xtea</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.Version.html\" title=\"struct rs3cache_backend::index::Version\">Version</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/error/enum.CacheError.html\" title=\"enum rs3cache_backend::error::CacheError\">CacheError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/error/struct.Decompression.html\" title=\"struct rs3cache_backend::error::Decompression\">Decompression</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/buf/enum.ReadError.html\" title=\"enum rs3cache_backend::buf::ReadError\">ReadError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/buf/struct.OpcodeNotImplemented.html\" title=\"struct rs3cache_backend::buf::OpcodeNotImplemented\">OpcodeNotImplemented</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.FileMissing.html\" title=\"struct rs3cache_backend::index::FileMissing\">FileMissing</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/decoder/enum.DecodeError.html\" title=\"enum rs3cache_backend::decoder::DecodeError\">DecodeError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/decoder/struct.Empty.html\" title=\"struct rs3cache_backend::decoder::Empty\">Empty</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/decoder/enum.DecodeError.html\" title=\"enum rs3cache_backend::decoder::DecodeError\">DecodeError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/decoder/struct.Empty.html\" title=\"struct rs3cache_backend::decoder::Empty\">Empty</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/io/error/struct.Error.html\" title=\"struct std::io::error::Error\">Error</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/error/enum.CacheError.html\" title=\"enum rs3cache_backend::error::CacheError\">CacheError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/error/struct.Io.html\" title=\"struct rs3cache_backend::error::Io\">Io</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/decoder/enum.DecodeError.html\" title=\"enum rs3cache_backend::decoder::DecodeError\">DecodeError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/decoder/struct.Unimplemented.html\" title=\"struct rs3cache_backend::decoder::Unimplemented\">Unimplemented</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.ArchiveMissing.html\" title=\"struct rs3cache_backend::index::ArchiveMissing\">ArchiveMissing</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, Error&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.Other.html\" title=\"struct rs3cache_backend::index::Other\">Other</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.Crc.html\" title=\"struct rs3cache_backend::index::Crc\">Crc</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.ArchiveMissing.html\" title=\"struct rs3cache_backend::index::ArchiveMissing\">ArchiveMissing</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/buf/enum.ReadError.html\" title=\"enum rs3cache_backend::buf::ReadError\">ReadError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/buf/struct.NotExhausted.html\" title=\"struct rs3cache_backend::buf::NotExhausted\">NotExhausted</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, <a class=\"struct\" href=\"https://docs.rs/serde_json/1.0.79/serde_json/error/struct.Error.html\" title=\"struct serde_json::error::Error\">Error</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/error/enum.CacheError.html\" title=\"enum rs3cache_backend::error::CacheError\">CacheError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/error/struct.JsonEncode.html\" title=\"struct rs3cache_backend::error::JsonEncode\">JsonEncode</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, <a class=\"enum\" href=\"rs3cache_backend/decoder/enum.DecodeError.html\" title=\"enum rs3cache_backend::decoder::DecodeError\">DecodeError</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/error/enum.CacheError.html\" title=\"enum rs3cache_backend::error::CacheError\">CacheError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/error/struct.Decode.html\" title=\"struct rs3cache_backend::error::Decode\">Decode</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.FileMissingNamed.html\" title=\"struct rs3cache_backend::index::FileMissingNamed\">FileMissingNamed</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/boxed/struct.Box.html\" title=\"struct alloc::boxed::Box\">Box</a>&lt;<a class=\"enum\" href=\"rs3cache_backend/buf/enum.ReadError.html\" title=\"enum rs3cache_backend::buf::ReadError\">ReadError</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/alloc/struct.Global.html\" title=\"struct alloc::alloc::Global\">Global</a>&gt;&gt;, <a class=\"enum\" href=\"rs3cache_backend/buf/enum.ReadError.html\" title=\"enum rs3cache_backend::buf::ReadError\">ReadError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/buf/struct.WithInfo.html\" title=\"struct rs3cache_backend::buf::WithInfo\">WithInfo</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/error/enum.CacheError.html\" title=\"enum rs3cache_backend::error::CacheError\">CacheError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/error/struct.Decompression.html\" title=\"struct rs3cache_backend::error::Decompression\">Decompression</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.ArchiveMissingNamed.html\" title=\"struct rs3cache_backend::index::ArchiveMissingNamed\">ArchiveMissingNamed</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.FileMissingNamed.html\" title=\"struct rs3cache_backend::index::FileMissingNamed\">FileMissingNamed</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/buf/enum.ReadError.html\" title=\"enum rs3cache_backend::buf::ReadError\">ReadError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/buf/struct.Eof.html\" title=\"struct rs3cache_backend::buf::Eof\">Eof</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/error/enum.CacheError.html\" title=\"enum rs3cache_backend::error::CacheError\">CacheError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/error/struct.Integrity.html\" title=\"struct rs3cache_backend::error::Integrity\">Integrity</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/io/error/struct.Error.html\" title=\"struct std::io::error::Error\">Error</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/buf/enum.ReadError.html\" title=\"enum rs3cache_backend::buf::ReadError\">ReadError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/buf/struct.FileSeek.html\" title=\"struct rs3cache_backend::buf::FileSeek\">FileSeek</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/index/enum.IntegrityError.html\" title=\"enum rs3cache_backend::index::IntegrityError\">IntegrityError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/index/struct.Version.html\" title=\"struct rs3cache_backend::index::Version\">Version</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/buf/enum.ReadError.html\" title=\"enum rs3cache_backend::buf::ReadError\">ReadError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/buf/struct.OpcodeNotImplemented.html\" title=\"struct rs3cache_backend::buf::OpcodeNotImplemented\">OpcodeNotImplemented</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, <a class=\"enum\" href=\"rs3cache_backend/buf/enum.ReadError.html\" title=\"enum rs3cache_backend::buf::ReadError\">ReadError</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/error/enum.CacheError.html\" title=\"enum rs3cache_backend::error::CacheError\">CacheError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/error/struct.Read.html\" title=\"struct rs3cache_backend::error::Read\">Read</a>"],["impl&lt;E: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/error/trait.Error.html\" title=\"trait core::error::Error\">Error</a>&gt; With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, E&gt;, <a class=\"enum\" href=\"rs3cache_backend/error/enum.CacheError.html\" title=\"enum rs3cache_backend::error::CacheError\">CacheError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/error/struct.Xtea.html\" title=\"struct rs3cache_backend::error::Xtea\">Xtea</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/buf/enum.ReadError.html\" title=\"enum rs3cache_backend::buf::ReadError\">ReadError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/buf/struct.NotNulTerminated.html\" title=\"struct rs3cache_backend::buf::NotNulTerminated\">NotNulTerminated</a>"],["impl With&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/convert/enum.Infallible.html\" title=\"enum core::convert::Infallible\">Infallible</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/io/error/struct.Error.html\" title=\"struct std::io::error::Error\">Error</a>&gt;, <a class=\"enum\" href=\"rs3cache_backend/decoder/enum.DecodeError.html\" title=\"enum rs3cache_backend::decoder::DecodeError\">DecodeError</a>&gt; for <a class=\"struct\" href=\"rs3cache_backend/decoder/struct.Gzip.html\" title=\"struct rs3cache_backend::decoder::Gzip\">Gzip</a>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()