# Authfix example: simple session

## Example
Setup a session based login and use the Api:
```sh
curl http://localhost:7080/public -i
```
Of course the public route is accessible.

Lets try the secured one:
```sh
curl http://localhost:7080/secured -i
```
Oh no, 401! You need to login 🤔

```sh
curl -c cookies.txt -X POST http://localhost:7080/login \
  -H "Content-Type: application/json" \
  -d '{ "email" : "test@example.org", "password": "password" }' \
  -i
```
Now try again
```sh
curl -b cookies.txt http://localhost:7080/secured -i
```

Logged in as Johnny 🎉

if you want to log out:

```sh
curl -c cookies.txt -b cookies.txt -X POST http://localhost:7080/logout -i
```

In this example, we use [CookieSessionStore](https://docs.rs/actix-session/latest/actix_session/storage/struct.CookieSessionStore.html) for simplicity. But it has a drawback, if you look at the logout command above:
the session is completely stored in the cookie, so if someone gets access to a valid cookie, he or she could impersonate the user.
You should use a server side store like [RedisSessionStore](https://docs.rs/actix-session/latest/actix_session/storage/struct.RedisSessionStore.html).
