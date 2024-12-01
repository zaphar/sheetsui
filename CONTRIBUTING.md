# Contributing to sheetui

The application is in the very early stages and it's a testament to the ratatui
and ironcalc crates that I was able to make as much progress as I have so far.
If you find this useful and have ideas you want to contribute to it then this
document outlines what little you need to know so far.

## CI/CD

There is no CI/CD to speak of yet and I certainly don't have full test coverage
although that is in progress. You can and should run cargo test to ensure you
haven't broken anything that currently has test coverage.

Ways you could help right away with this might be:

* Help setting up CI/CD in github
* More test coverage

## Feature Proposals.

I have no formal process for this at the moment. But if there is a feature you
would like to add then starting out with an issue to discuss it is a great way
to start. Prototypes on your own fork are certainly encouraged as well.

## Bug reports

If you spot a bug and can replicate it with a simple sheet and ui interations
then generating a log of the inputs, using `--log-input path/to/file` as well
as a spreadsheet to replicate it against is a great way to capture information
to attach to the issue.
