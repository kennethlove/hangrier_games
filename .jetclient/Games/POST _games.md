```toml
name = 'POST /games'
description = 'Create a new game'
method = 'POST'
url = '{{Host}}{{global.GamesApi}}'
sortWeight = 2000000
id = 'eed6e8f5-4869-404b-be66-19be9347db08'
```

### Example

```toml
name = 'Without name'
id = '008f0158-536a-4347-a833-32fafddd89e8'
```

### Example

```toml
name = 'With name'
id = 'a487f7a2-c5d2-44af-9abd-5a2b71ad04cf'

[body]
type = 'JSON'
raw = '{"name":  "test-name"}'
```
