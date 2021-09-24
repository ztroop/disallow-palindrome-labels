# disallow-palindrome-labels

## Summary

This is a Kuberwarden policy to reject pods that have one or more label keys that are palindromes. This is demonstration around `kubewarden` written in Rust and Wasm!

### Go Versus Rust

After studying Kubewarden, I definitely see that Go may not be the best choice for plugin implementation. Rust on the other hand, fully supports WebAssembly as a first-class citizen.

> The official Go compiler cannot produce WebAssembly binaries that can be run outside of the browser. If we really wanted to use Go, we would have to use the TinyGo compiler.

> TinyGo doesn't yet support all the Go features (see here to see the current project status). Currently its biggest limitation is the lack of a fully supported reflect package. That leads to the inability to use the encoding/json package against structures and user defined types.

https://docs.kubewarden.io/writing-policies/go/01-intro.html

To avoid _yak-shaving_, I decided to use Rust for this demonstration to keep it simple.

## Operational Example

### Allowed Pod Creation

```yaml
apiVersion: v1
kind: Pod
metadata:
name: hello-world
  labels:
    env: production
spec:
  containers:
  - name: nginx
  image: nginx
```

### Rejected Pod Creation

```yaml
apiVersion: v1
kind: Pod
metadata:
name: hello-world
  labels:
    env: production
    level: debug
spec:
  containers:
  - name: nginx
  image: nginx
```

## Testing

Testing can be invoked by using the `cargo test` or `make test` commands. In this particular policy, we're verifying manifests that have valid, invalid labels and no labels at all.

## Continuous Integration

The `test.yml` workflow in the `.github/workflows` folder would normally cover error detection, testing, formatting and linting. However, this is a **private** repository and would require a paid GitHub account to facilitate CI operations.

## Settings

This policy has no configurable settings. Though, this would be fairly easy to implement in `settings.rs` if we wanted to expand upon the functionality.
