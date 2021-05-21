# Testing

This is just a general overview what what to expect as there is nothing formal to run tests with.

 - create, update, and delete journal entries
 - create, update, and delete custom fields
   - when deleting a custom field, all associated data attached to that field should also be deleted.
   - changing the type of a field has not been formally tested and will create some wierd bevaiour or just error out.
 - create, update, and delete custom tags
   - when deleting tags, all associated data attached to that tag should also be deleted
 - update user information without error. email is currently not used for anything but is required to exist in some form. with that no formal validation is done on the email string to make sure it is valid or even exists
 - admins can create users and managers.
   - a user can be assigned to multiple managers
   - a manager can be assigned to multiple users
 - only a manager with read permissions on the requested user can access their information.
 - only a user is allowed to edit their entries

will put more down as they come up