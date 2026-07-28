# XML converter

This local crate contains the XML-to-JSON converter used by fakESB. Its public
API is `xml::convert::to_json`, which preserves the XML attribute and element
shape expected by the ESB request parser.

The converter was embedded from the internal `xml` crate at commit
`c08f095eb2a9938a5b662a056d9f2d4e3158fd90` so public builds do not require a
private Git repository.

The crate is distributed under the Apache License, Version 2.0. See the
repository `LICENSE` file.
