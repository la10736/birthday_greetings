FROM openjdk:7-alpine

ENV JARS=jars
COPY fakeSMTP-2.0.jar ${JARS}/

ENV PORT=2525
ENV HOST=0.0.0.0
ENV OUTPUT=/mailout/

RUN mkdir -p $OUTPUT

ENTRYPOINT java -jar ${JARS}/fakeSMTP-2.0.jar -s -b -p $PORT -o $OUTPUT -a $HOST
