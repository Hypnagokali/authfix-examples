# Authfix example: session with mfa (Authenticator)

## Example

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

We get the code via a helper route `/code` and send it then to the mfa-endpoint:

```sh
curl -c cookies.txt -b cookies.txt -X POST -i http://localhost:7080/login/mfa \
    -H "Content-Type: application/json" \
    -d "{\"code\": \"$(curl -s http://localhost:7080/code)\"}" \
    -i
```

We should be now logged in
```sh
curl -c cookies.txt -b cookies.txt http://localhost:7080/secured -i
```

