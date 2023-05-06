# Jirust - WIP
A terminal UI for jira written in rust.

An application with developers and engineers in mind.  It is solely focused on updating tickets at the moment.
![jirust](https://user-images.githubusercontent.com/7011993/225179809-b4683ea5-93e5-4c4c-abf5-e6534df0f5a3.gif)

## Important notices
* This is currently tested with JIRA cloud.
* I'm (Author: Mo Ali) am an infrastructure engineer by trade.  This is my first programming project that I'm sharing out.  This is also my first rust project and am using it to learn rust.  You can watch my development on [twitch](https://www.twitch.tv/mo_ali141)

## Current requirements
These are environment variables required to use this tool.
---
* JIRA_USER_EMAIL: "example@examplerserver.com" 
* JIRA_API_KEY: "abcdefghijklmnopqrstuvwxyz1234567890"
* JIRA_DOMAIN: "https://mycompany.atlassian.net"

## Current capabilities
* List projects
* Filter/Search projects (Search JIRA API if not found within pagination limit)
* List tickets
* List ticket details such as labels, components, description, and parent ticket even if parent is another ticket or epic.
* Filter/Search ticket (Search JIRA API if not found within pagination limit)
* List ticket comments
* Add comments to ticket
* Move ticket to another status (ex: To do -> In Progress)

## Default keys
* Filter/Search: "/"
* Help: "?"

TODO:
- [X] UI to generate a list of all projects
- [X] Pagination for projects greater than the max limit returned by JIRA rest API
- [X] UI to list all issues/tasks on selected project
- [X] Pagination list issues greater than the max limit returned by JIRA rest API
- [X] UI to view selected issue details
- [ ] POC support for JIRA data types such as tables, list, and code blocks using atlassian document format
- [ ] Add functionality to support ticket sorting by sprint
- [ ] Only view specified ticket status
- [ ] Only view tickets assigned to specific user
