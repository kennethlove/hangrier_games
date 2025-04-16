```toml
name = 'POST /users'
description = 'Create a new user'
method = 'POST'
url = '{{Host}}/api/users'
sortWeight = 1000000
id = '52bac3bf-210b-4dc6-9d16-67b339a16a6f'

[body]
type = 'JSON'
raw = '''
{
  "email": "kennethlove@gmail.com",
  "pass": "password2"
}'''
```
