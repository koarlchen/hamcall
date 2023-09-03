# Ham Radio Callsign Analyzer

The library implements a parser for the ClubLog XML file. Based on the data, an analyzer for a callsign is implemented to get further information like the name of the entity, the ADIF identifier or the continent. Fur further information have a look at the file [call.rs](src/call.rs).

Both modules offer a few basic tests that assume a ClubLog XML file to be available under `data/clublog/cty.xml`.


Analyzing callsigns is not that easy. Next to obvious callsigs like `DL1ABC` or `F/DL1ABC` there are also tricky ones like `SV1ABC/A` (SV/A is Mount Athos, SV would be just Greece), `CE0Y/PG5M` (which of both parts is the prefix?) or `F0BAU/FC` (prefix is in the back!). The given implementation should handle quite a few of those calls but will never be assumed to analyze all callsigns, specifically the quite special ones, correctly. Just keep that in your mind when using the library. If you are curious of what type of special calls are covered, just have a look in the tests within the file [call.rs](src/call.rs).

If you come across a callsign where the library returns an unexpected entity or the call analysis returns an error, first have a look into the ClubLog XML file yourself and check if the callsign should be valid according to the entries there. If so, open an issue with the callsign you expect to be mistakenly analyzed the wrong way together with the timestamp, the date of the ClubLog XML and according to which entry of the ClubLog XML file you would expect different information.