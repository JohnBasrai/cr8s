#!/bin/bash
set -euo pipefail

progname="$(basename "$0")"

echo "$progname: 🧹 Docker system cleanup - removing dangling images and exited containers"

# Clean up dangling images (untagged images with <none>:<none>)
echo "$progname: 🗑️  Removing dangling images..."
dangling_images=$(docker images --filter "dangling=true" --quiet)

if [ -n "$dangling_images" ]; then
    echo "$progname: Found dangling images, removing..."
    docker rmi $dangling_images
    echo "$progname: ✅ Dangling images removed"
else
    echo "$progname: ✅ No dangling images found"
fi

# Clean up exited containers
echo "$progname: 🗑️  Removing exited containers..."
exited_containers=$(docker ps --filter "status=exited" --quiet)

if [ -n "$exited_containers" ]; then
    echo "$progname: Found exited containers, removing..."
    docker rm $exited_containers
    echo "$progname: ✅ Exited containers removed"
else
    echo "$progname: ✅ No exited containers found"
fi

# Optional: Clean up unused networks (commented out for safety)
# echo "$progname: 🌐 Removing unused networks..."
# docker network prune -f

# Optional: Clean up unused volumes (commented out for safety)
# echo "$progname: 💾 Removing unused volumes..."
# docker volume prune -f

echo "$progname: 🎉 Docker cleanup completed!"
echo "$progname: 💡 For more aggressive cleanup, consider:"
echo "$progname:     docker system prune        # Remove all unused data"
echo "$progname:     docker system prune -a     # Remove all unused data including images"