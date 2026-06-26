FROM nginx:alpine

RUN printf 'server {\n\
  listen 80;\n\
  root /usr/share/nginx/html;\n\
  index index.html;\n\
  location / { try_files $uri $uri/ /index.html; }\n\
  location /downloads/ {\n\
    types { application/x-apple-diskimage dmg; }\n\
    default_type application/octet-stream;\n\
    add_header Content-Disposition "attachment";\n\
  }\n\
}\n' > /etc/nginx/conf.d/default.conf

COPY apps/web/ /usr/share/nginx/html
RUN rm -f /usr/share/nginx/html/Dockerfile

EXPOSE 80
