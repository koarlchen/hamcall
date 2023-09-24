# Ham Radio Callsign Analyzer

The library implements a parser for the ClubLog XML file.
Based on the data, an analyzer for callsigns is implemented to get further information like the name of the entity, the ADIF identifier or the continent.

Simple applications within the folder `examples/` show the basic usage of the library.
Next to that, both modules offer a few tests.
These tests assume a ClubLog XML file to be available under `data/clublog/cty.xml`.
While developing and testing the library, the `cty.xml` with the datestring `2023-09-22T07:31:24+00:00` was used in combination with the XML schema file `cty.xsd` downloaded on the same day.
Future changes to the files may break the given code.
Apparently, the files are not versioned except for the timestamp within the `cty.xml`.
On how to obtain a `cty.xml` or rather an API key just have a look at the ClubLog website.

Analyzing callsigns is not that easy.
Next to obvious callsigns like `DL1ABC` or `F/DL1ABC` there are also tricky ones like `SV1DC/A` (SV/A is Mount Athos, SV is Greece. This exact callsign is valid for Mount Athos, but for example `SV1ABC/A` does not have to be Mount Athos), `CE0Y/PG5M` (which of both parts is the prefix?) or `F0BAU/FC` (prefix is in the back!).
The given implementation should handle quite a few of those calls but will never be assumed to analyze all callsigns, especially the quite special ones, correctly.
For example, `LS4AA/F` counts for Argentina, even though the additional `/F` indicates France.
Such calls cannot be covered by using solely the ClubLog XML data.
Just keep that in your mind when using this library.
If you are curious about what type of special calls are covered, just have a look at the tests within the file [call.rs](src/call.rs).

If you come across a callsign where the library returns unexpected information (e.g. the wrong entity or continent) or the call analysis returns an error, first have a look into the ClubLog XML file yourself and check your callsign against the information there.
If you were able to find an entry that leads to a different interpretation of the callsign, open an issue with the callsign you expect to be mistakenly analyzed the wrong way together with the timestamp, the date of the ClubLog XML and according to which entry you would expect different information.