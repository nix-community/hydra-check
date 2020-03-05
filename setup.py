from setuptools import setup

setup(
    name="hydra-check",
    description="check hydra for the build status of a package",
    version="1.0.0",
    packages=["hydracheck"],
    license="MIT",
    long_description=open("README.md").read(),
    author="Felix Richter",
    author_email="github@krebsco.de",
    install_requires=[
        "requests",
        "beautifulsoup4",
        "docopt"
    ],
    entry_points={"console_scripts": ["hydra-check = hydracheck.cli:main"]},
    classifiers=[
        "Intended Audience :: Human",
        "Natural Language :: English",
        "Operating System :: POSIX :: Linux",
        "Development Status :: 3 - Alpha",
        "Programming Language :: Python",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: Implementation :: CPython",
    ],
)
