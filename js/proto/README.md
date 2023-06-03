# yab/proto

Prototype implementation of the bundler in javascript. The long-term plan is of course to use Rust, but it's not a great prototyping language.

This package is pretty much at a standstill because even though it can parse import statements, in order to accurately parse export statements and support ESM, I'm going to have to write a full-featured JS parser, so this is turning into a boil-the-ocean project pretty quickly.
