docker run --network=host -u0 -it /home/dev --name rust-dev-pw \
  ghcr.io/johnbasrai/cr8s/rust-dev:1.83.0-rev6 bash
```

#### 2. **Inside the container: install deps and Playwright**

```bash
cd /home/dev/

# Update and install required browser deps
apt update && apt upgrade -y
apt install -y \
  libnspr4 libnss3 libatk1.0-0 libatk-bridge2.0-0 \
  libcups2 libxkbcommon0 libatspi2.0-0 libxdamage1 \
  libcairo2 libpango-1.0-0 libasound2

# Optional: install git if needed by npm packages
apt install -y git

# Install Playwright test runner
npm install --save-dev @playwright/test

# Download browser binaries
npx playwright install

# Optional final upgrade pass
apt upgrade -y

exit
```

#### 3. **Save your patched container**

```bash
docker commit rust-dev-pw rust-dev-1.83.0-rev6-pw
docker save rust-dev-1.83.0-rev6-pw > rust-dev-1.83.0-rev6-pw.tar
```

> You can later load it on another system with:

```bash
docker load < rust-dev-1.83.0-rev6-pw.tar
```

---

Let me know if you'd like to:

* Tag and push it to GHCR
* Run Playwright tests from this container via script
* Or integrate this into a `Dockerfile.pw` for reproducibility
