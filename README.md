# Intention

This is a 'breakable toy' intended to provide a practical, personal vehicle with which to explore writing programs in Rust. The goal is to parse sources for PC video games into a unified model that can then be used to launch those games.

# Sources

* Trove - The Humble Bundle Monthly Trove
* Steam - Parse VDF files to get list of all games in the local library.
* Monthly - Humble Bundle Monthly titles
* Ubisoft
* Epic

# Status

Tarnish is very much a work in progress and is in its early days. Do not expect this to be of use to anyone else at the moment.

Work is underway to parse the Humble Bundle Monthly Trove titles. Games are periodically removed from the trove and I'd like to have them and their metadata downloaded before that happens.

# TODO

* Parse command line arguments to enable different modes of operation.
* Externalize settings into a TOML file in order to avoid checking in user specific details.
* Create a master list of trove games. Separate it from the parsing / manipulation of the trove web assets.
* Create multiple crates to split out the functionality.
* Port Go steam VDF parser to Rust.

