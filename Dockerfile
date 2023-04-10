FROM node:18-alpine

WORKDIR /app
COPY run.sh ./
COPY client/dist ./client/dist
COPY client/package*.json client/config.env ./client/
RUN sed -i "s/IS_PRODUCTION=false/IS_PRODUCTION=true/" ./client/config.env

RUN apt-get update
RUN npm install -g n
RUN n lts

RUN cd ./client && npm install && cd ../

EXPOSE 4000

ENTRYPOINT ["./run.sh"]
