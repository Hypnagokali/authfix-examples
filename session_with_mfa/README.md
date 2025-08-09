# Authfix example: session with mfa (Authenticator)

## Example 1: Login with an authenticator

Lets log in like before:
```sh
curl -c cookies.txt -X POST http://localhost:7080/login \
  -H "Content-Type: application/json" \
  -d '{ "email" : "test@example.org", "password": "password" }' \
  -i
```

The response indicates, that you need a second factor to login and what type of factor:

```sh
{"status":"MfaNeeded","mfaId":"TOTP_MFA"}
```

You get the code via a helper route `/code` and send it then to the mfa-endpoint:

```sh
curl -c cookies.txt -b cookies.txt -X POST -i http://localhost:7080/login/mfa \
    -H "Content-Type: application/json" \
    -d "{\"code\": \"$(curl -s http://localhost:7080/code)\"}" \
    -i
```

You are now logged in
```sh
curl -b cookies.txt http://localhost:7080/secured -i
```

## Example 2: Register an authenticator

Start the application and go to http://localhost:7080/qr-code
You can register the secret and verify it.

