# Tasker
Uses [Ratatui](https://ratatui.rs/) and [Crossterm](https://github.com/crossterm-rs/crossterm) to render a simple text-based task manager.

### The issues with writing UIs:
* Whenever UI code is written, there is a tendency to write nested conditionals many, many levels deep.
* The UI main loop tends to get very long and unwieldy.
* Every new feature means new conditionals and deeper nested ifs.

The aim with this project is to write a UI with the least amount of cognitive overhead possible. That means favoring readability
over flatness, but striving for flatness where possible.

### License:
This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.