```toml
name = 'POST /games'
description = 'Create a new game'
method = 'POST'
url = '{{Host}}{{global.GamesApi}}'
sortWeight = 2000000
id = 'eed6e8f5-4869-404b-be66-19be9347db08'

[auth.bearer]
token = 'eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJpYXQiOjE3NDQ4NDE0OTksIm5iZiI6MTc0NDg0MTQ5OSwiZXhwIjoxNzQ3NDMzNDk5LCJpc3MiOiJTdXJyZWFsREIiLCJqdGkiOiJjNzljYzc3Yi1iZDJjLTQ5OTktOWUwOC04ZjFkNDcwMTM3OGMiLCJOUyI6ImhhbmdyeS1nYW1lcyIsIkRCIjoiZ2FtZXMiLCJBQyI6ImFjY291bnQiLCJJRCI6InVzZXI6aHFvNmxiZTI4aXlvMnRtcjMyaXgifQ.BI7vYiFIW-1cczr-wqZxGdLC_JB69fQkCG-EMoAZaCu2gzasJfhv_a4Zf_I8SJtpMV0E-QGNg8k5HSODwywSVw'

[body]
type = 'JSON'
raw = '''
{"name":  "johnny-apple-cakes",
  "identifier":  "uuid-0000-000001",
  "status":  "NotStarted"}'''
```

### Example

```toml
name = 'Without name'
id = '008f0158-536a-4347-a833-32fafddd89e8'

[body]
type = 'JSON'
```

### Example

```toml
name = 'With name'
id = 'a487f7a2-c5d2-44af-9abd-5a2b71ad04cf'

[body]
type = 'JSON'
raw = '{"name":  "test-name"}'
```
