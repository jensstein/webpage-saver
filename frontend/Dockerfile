FROM node:24-bookworm-slim

RUN apt-get update && \
	apt-get -y upgrade && \
	apt-get install -y curl

USER node
WORKDIR /home/node

COPY --chown=node components components
COPY --chown=node helpers helpers
COPY --chown=node next.config.js next.config.js
COPY --chown=node package.json package.json
COPY --chown=node yarn.lock yarn.lock
COPY --chown=node app app
COPY --chown=node middleware.js middleware.js
COPY --chown=node postcss.config.mjs postcss.config.mjs
COPY --chown=node public public
COPY --chown=node requests requests
COPY --chown=node styles styles
COPY --chown=node download-fonts.sh download-fonts.sh

ENV YARN_CACHE_FOLDER=/tmp/yarn-cache
ENV PATH=/home/node/.local/bin:$PATH

RUN mkdir -p fonts && \
	./download-fonts.sh && \
	mkdir -p $HOME/.local/bin && \
	corepack enable --install-directory /home/node/.local/bin && \
	yarn install --immutable && \
	yarn build && \
    rm -rf /tmp/yarn-cache

CMD ["yarn", "start"]

EXPOSE 3000

ENV NEXT_TELEMETRY_DISABLED=1

LABEL BACKEND_URL 'url for backend server'
