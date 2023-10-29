# Ham Radio Callsign Analyzer

The library implements a parser for the ClubLog XML file.
Based on the data, an analyzer for callsigns is implemented to get further information like the name of the entity, the ADIF identifier or the continent.


## Usage

Simple applications within the folder `examples/` show the basic usage of the library.
Next to that, both modules offer a few tests.
These tests assume a ClubLog XML file to be available under `data/clublog/cty.xml`.
While developing and testing the library, the `cty.xml` with the datestring `2023-09-22T07:31:24+00:00` was used in combination with the XML schema file `cty.xsd` downloaded on the same day.
Future changes to the files may break the given code.
Apparently, the files are not versioned except for the timestamp within the `cty.xml`.
On how to obtain a `cty.xml` or rather an API key just have a look at the ClubLog website.


## Callsign Analysis

Analyzing callsigns is not that easy.
Next to obvious callsigns like `DL1ABC` or `F/DL1ABC` there are also tricky ones like `SV1DC/A` (SV/A is Mount Athos, SV is Greece. This exact callsign is valid for Mount Athos, but for example `SV1ABC/A` does not have to be Mount Athos), `CE0Y/PG5M` (which of both parts is the prefix?) or `F0BAU/FC` (prefix is in the back!).
The given implementation should handle quite a few of those calls but will never be assumed to analyze all callsigns, especially the quite special ones, correctly.
You may have a look at the section of known limitations below.
If you are curious about what type of special calls are covered, just have a look at the tests within the file [call.rs](src/call.rs).

After all, the entity named on the received QSL card should be deemed to be the correct one. You may also use the online callsign analyzer of ClubLog directly. Even though, the data used here is provided by them, they sometimes have more information that is not part of the XML file.


## Error Reports

If you come across a callsign where the library returns unexpected information (e.g. the wrong entity or continent) or the call analysis returns an error, first have a look into the ClubLog XML file yourself and check your callsign against the information there.
If you were able to find an entry that leads to a different interpretation of the callsign, open an issue with the callsign you expect to be mistakenly analyzed the wrong way together with the timestamp, the date of the ClubLog XML and according to which entry you would expect different information.

## Known Limitations

- `LS/A`:
  Callsigns from Argentina may use a single character appendix, e.g. `LS4AA/F`.
  This may be true for other countries as well.
  In this special case the additional `/F` indicates the entity France which will actually be returned by the callsign analysis routine.
  The fact, that these calls are operated from Argentia cannot be covered by using solely the ClubLog XML data and would require more specialized rules exceeding the information available within the XML file.
- `3D2/R`, `SV/A`:
  The prefix list of the ClubLog XML contains special entries like `SV/A` or `3D2/R`.
  As of now the interpretation of these prefixes is not as clear as it should be.
  As noted above the call `SV1ABC/A` may not be valid for Mount Athos (`SV/A`).
  Since Mount Athos is currently whitelisted, this is not such a big deal, but not all special prefixes reference entites that are whitelisted.
  Furthermore, special prefixes, where both parts of the prefix are valid prefixes for itself are not covered correctly in all cases.
  For example `3D2` references Fiji Islands and `R` references European Russia but both parts together reference Rotuma (`3D2/R`).
  Calls like `3D2ABC/R` are correctly mapped to Rotuma but calls like `3D2/W1ABC/R` will currently lead to an error, since the call contains three potential valid prefixes (`3D2`, `W` and `R`) which is considered to be invalid by the current implementation.
  Maybe this type call is after all not valid, but thats my current interepretation of the prefix `3D2/R`.