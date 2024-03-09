FROM node:alpine

USER node
WORKDIR /home/node

COPY --chown=node components components
COPY --chown=node helpers helpers
COPY --chown=node next.config.js next.config.js
COPY --chown=node package.json package.json
COPY --chown=node yarn.lock yarn.lock
COPY --chown=node app app
COPY --chown=node middleware.js middleware.js
COPY --chown=node tailwind.config.js tailwind.config.js
COPY --chown=node postcss.config.js postcss.config.js
COPY --chown=node public public
COPY --chown=node requests requests
COPY --chown=node styles styles

ENV YARN_CACHE_FOLDER=/tmp/yarn-cache

RUN yarn install --frozen-lockfile && \
	yarn build && \
    rm -rf /tmp/yarn-cache

CMD ["yarn", "start"]

EXPOSE 3000

ENV NEXT_TELEMETRY_DISABLED=1

LABEL BACKEND_URL 'url for backend server'
