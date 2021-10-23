try:
    from setuptools import setup
except ModuleNotFoundError as e:
    raise ModuleNotFoundError("You must install the 'setuptools' package. Try 'pip install setuptools'.") from e

try:
    from setuptools_rust import Binding, RustExtension
except ModuleNotFoundError as e:
    raise ModuleNotFoundError("You must install the 'setuptools-rust' package. Try 'pip install setuptools-rust'.") from e

setup(
    name="rs3",
    version="1.0",
    rust_extensions=[RustExtension("rs3.rs3", binding=Binding.PyO3)],
    packages=["rs3"],
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
)
