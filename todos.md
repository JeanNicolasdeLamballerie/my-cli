# CLI todos

## List of features to add & small fixes

# IMPORTANT


- [x] add a view command. it should be as easy as "rush show todos"
- I really need to figure out this "are we in a project or not" problem at the root...


# For the todo feature : 

- [x] Test if the file is properly removed for the basic usage
- [x] add a confirmation screen to "delete all" 
- [x] make the self.log.push & self.refresh a single method to call with the object to push
- [ ] fix the refresh itself to keep (or not) the newly added todos
- [ ] wording : newly added todos should say "discard" instead of delete
- [ ] maybe fix the position of windows on refresh (not sure what strategy to go for, could reset it)


- [ ] minor but some bounding windows are too close to the buttons. Might need to use show_inside()
- [ ] add some color ?
- [ ] maybe add a way to fold all the todos at once

