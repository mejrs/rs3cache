try:
    from setuptools import setup
except ModuleNotFoundError as e:
    raise ModuleNotFoundError("You must install the 'setuptools' package. Try 'pip install setuptools'.") from e

try:
    from setuptools_rust import Binding, RustExtension
except ModuleNotFoundError as e:
    raise ModuleNotFoundError("You must install the 'setuptools-rust' package. Try 'pip install setuptools-rust'.") from e

setup(
    name="osrs",
    version="1.0",
    rust_extensions=[RustExtension("osrs.osrs", binding=Binding.PyO3)],
    packages=["osrs"],
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
)
