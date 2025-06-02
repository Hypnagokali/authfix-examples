# Authfix example: configure the session

The session based authentication of Authfix is based on [Actix-Session](https://docs.rs/actix-session/latest/actix_session/).

## Example

Login with curl:
```sh
curl -c cookies.txt -X POST http://localhost:7080/login \
  -H "Content-Type: application/json" \
  -d '{ "email" : "test@example.org", "password": "password" }' \
  -i
```
Now the `Set-Cookie` header should look like this:
> set-cookie: **sessionId**=QWP%2Fl+qkWHE8kn2ZqvuCnZIvwLX5r0sSzYI8Xmd9fCgeKn5f0G8K1GdWaL7jzwUzeNr3v4sz6%2F30TeaMxY7Qcf7ilKsS; HttpOnly; **SameSite=Lax**; Path=/;  **Max-Age=86400**

And no secure flag.

Of course we can access the secured resource:
```sh
curl -b cookies.txt http://localhost:7080/secured -i
```

