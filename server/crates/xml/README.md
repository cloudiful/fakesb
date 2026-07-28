# XML converter

This local crate contains the XML-to-JSON converter used by the HTTP mock
service. Its public API is `xml::convert::to_json`, preserving XML attributes,
nested elements, repeated elements, and text values in a JSON representation.

The crate is distributed under the Apache License, Version 2.0. See the
repository `LICENSE` file.
