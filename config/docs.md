$ openssl genrsa -out config/security/keys/access_key.pem 4096
$ openssl rsa -in config/security/keys/access_key.pem -pubout -out config/security/keys/access_key_pub.pem

The first command will generate a private key and store it inside the access_key.pem file. The 2nd command will extract the public key from the private key file and write it to the access_key_pub.pem file.

Now to generate the refresh keys just repeat the same command but replace all occurrences of access with refresh.

AppContext

In a web application with JWT authentication, we need several shared resources that are:

    Expensive to create - Database connection pools, Redis connections, and cryptographic keys take time to initialize.

    Used across many requests - Every authenticated request needs access to JWT keys, database, and Redis.

    Thread-safe and clonable - Web servers handle requests concurrently, so these resources must be safely shared.

The AppContext struct solves the above issues by bundling these resources together.