# JPEG Metadata Extractor

A command-line tool written in Rust that extracts metadata from JPEG images and outputs it in JSON format.

TODO:
- Opening each JPEG twice was a bit of an oversight on my part. Ideally I would have refactored things a bit to only open it once.

- I'm not particularly happy with the level of validation on the input files. I almost missed that I wasn't checking if the file exists, and had to quickly add it in last minute. But it would have been nice to have checks for corrupt files, or that the file is readable and verify permissions.

- Having test images included in source control isn't ideal, but thought it was better for testing purposes than excluding them.