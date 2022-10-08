TODO:
====

* add ability to restrict tasks based on hostname. should be pretty simple.

version 0.7.4
=============

* add function to get latest log file, to be passed to your editor (like so: `nvr (hm log)`)
* updates to the arg parsing timing so it parses args before it does anything (no more writing log files for just having run `hm clean`)

version 0.7.1
=============

* clippy lint cleanup

version 0.7.0
=============

* not 0.6.8 because our commandline args have changed and we have some new args to lib functions.
* now supports passing -t to specify a task (and its dependencies) to complete.
  - this runs ONLY that subtree.
* now REQUIRES -c or --config to manually specify config location.
