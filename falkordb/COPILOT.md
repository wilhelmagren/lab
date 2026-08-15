### User Roles

Define two primary roles:
Role	Access Type	Graphs Accessible
data_engineer	Read + Write	All eng_graph_*
analyst	Read-only	All eng_graph_*, shared_*

FalkorDB runs on redis, so we'll use Redis ACLs to enforce permissions.

```
# Create users
ACL SETUSER data_engineer on >password ~eng_graph_* +GRAPH.QUERY +GRAPH.PROFILE +GRAPH.EXPLAIN
ACL SETUSER analyst on >password ~eng_graph_* ~shared_graph_* +GRAPH.QUERY
```

~eng_graph_*: limits access to specific graph keys

+GRAPH.QUERY: allows querying

+GRAPH.PROFILE, +GRAPH.EXPLAIN: useful for engineers

Analysts don’t get write permissions (GRAPH.DELETE, GRAPH.CONFIG, etc.)

### Enable Redis Logging

Configure Redis to log commands:
```bash
loglevel notice
logfile /var/log/redis/falkordb.log
```

### Security Best Practices

Rotate passwords regularly for ACL users

Use TLS to encrypt Redis traffic

Restrict network access to FalkorDB server via firewall or VPN

Backup graphs regularly using Redis persistence (RDB or AOF)


### Redis ACL configration

Data Engineers:
```bash
ACL SETUSER data_engineer on >password \
  ~eng_graph_* \
  +GRAPH.QUERY +GRAPH.PROFILE +GRAPH.EXPLAIN +GRAPH.DELETE +GRAPH.CONFIG
```

Analysts:

```bash
ACL SETUSER analyst on >password \
  ~ana_graph_* ~eng_graph_* \
  +GRAPH.QUERY +GRAPH.PROFILE +GRAPH.EXPLAIN \
  +GRAPH.DELETE +GRAPH.CONFIG \
  -@write ~eng_graph_*  # deny write on eng_graph_*
```

This setup:

Grants full R+W access to ana_graph_*

Grants read-only access to eng_graph_*

Prevents accidental writes to data engineer graphs

If you want finer granularity, you can create per-user ACLs or use ACL categories to group permissions.


```
/falkordb-setup/
│
├── create_graph.sh
├── graph_registry.json
└── acl_templates/
    ├── data_engineer.acl
    └── analyst.acl
```

create_graph.sh

```bash
#!/bin/bash

# Usage: ./create_graph.sh <graph_name> <role> <username>

GRAPH_NAME=$1
ROLE=$2
USERNAME=$3

# Prefix based on role
if [ "$ROLE" == "data_engineer" ]; then
    PREFIX="eng:"
elif [ "$ROLE" == "analyst" ]; then
    PREFIX="ana:"
else
    echo "Invalid role. Use 'data_engineer' or 'analyst'."
    exit 1
fi

# Full graph key
GRAPH_KEY="${PREFIX}${GRAPH_NAME}"

# Create graph (dummy query to initialize)
redis-cli GRAPH.QUERY "$GRAPH_KEY" "CREATE ()"

# Assign ACL
if [ "$ROLE" == "data_engineer" ]; then
    ACL_CMD="ACL SETUSER $USERNAME on >password ~eng:* +GRAPH.QUERY +GRAPH.PROFILE +GRAPH.EXPLAIN +GRAPH.DELETE +GRAPH.CONFIG"
elif [ "$ROLE" == "analyst" ]; then
    ACL_CMD="ACL SETUSER $USERNAME on >password ~ana:* ~eng:* +GRAPH.QUERY +GRAPH.PROFILE +GRAPH.EXPLAIN +GRAPH.DELETE +GRAPH.CONFIG -@write ~eng:*"
fi

# Apply ACL
redis-cli "$ACL_CMD"

echo "Graph '$GRAPH_KEY' created and ACL assigned to user '$USERNAME'."
```

Redis Key Prefix Isolation

Using prefixes like eng: and ana: ensures:

Logical separation of graphs

Easier ACL scoping (~eng:*, ~ana:*)

Cleaner monitoring and backups

You can even use Redis keyspace notifications to track changes per prefix.
