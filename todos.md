# CLI todos

## List of features to add & small fixes

# IMPORTANT


- [x] add a view command. it should be as easy as "rush show todos"
- I really need to figure out this "are we in a project or not" problem at the root...

# General improvement

# For the todo feature : 

- [x] Test if the file is properly removed for the basic usage
- [x] add a confirmation screen to "delete all" 
- [x] make the self.log.push & self.refresh a single method to call with the object to push
- [ ] fix the refresh itself to keep (or not) the newly added todos
- [x] wording : newly added todos should say "discard" instead of delete
- [ ] maybe fix the position of windows on refresh (not sure what strategy to go for, could reset it)


- [ ] minor but some bounding windows are too close to the buttons.
- [ ] add some color ?
- [ ] maybe add a way to fold all the todos at once
- [ ] Make the editor scrollable.

# SECURITY 

- [x] ~~Swap the dash util to use pwsh for proper input sanitization (!!)~~
- [x] ~~Swap to change_current_dir() instead (how did I miss this function ?)~~
- [x] I didn't miss it, change_current_dir() is only for the process. Updated to use Invoke-Expression on Powershell to avoid cmd injections
