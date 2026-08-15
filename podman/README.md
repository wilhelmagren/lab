# Podman

**Podman** is a tool for managing containers, pods, and images, allowing you to seamlessly
work with containers and Kubernetes from your local environment. Podman is daemonless,
secure through its rootless containers, open source first and won't lock you in, and
Kubernetes ready.

```
brew install podman
```

After installing you need to create and start your first Podman machine:

```
podman machine init
podman machine start
```

You can then verify the installation information with:

```
podman info
```

Podman basically does what Docker does, that's it, but its free and open-sourced software
:)

