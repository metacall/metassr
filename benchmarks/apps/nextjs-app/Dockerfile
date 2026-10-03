# Next.js benchmark app image: the Next.js side of the metassr vs Next.js
# benchmark. Installs `next` from npm and serves the production build.
#
# Build with the app directory as context:
#   docker build -f benchmarks/apps/nextjs-app/Dockerfile -t metassr-bench-nextjs benchmarks/apps/nextjs-app
#
# NODE_VERSION pins the base image; NEXT_VERSION pins the Next.js package.

ARG NODE_VERSION=22-trixie-slim
ARG NEXT_VERSION=latest

FROM node:${NODE_VERSION}

ARG NEXT_VERSION

WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .

# Allow overriding next's version at build time without touching the lockfile.
RUN if [ "${NEXT_VERSION}" != "latest" ]; then \
		npm install "next@${NEXT_VERSION}"; \
	fi && npm run build

EXPOSE 8080

CMD ["npm", "start"]