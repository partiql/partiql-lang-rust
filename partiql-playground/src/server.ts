const express = require('express');
const favicon = require('serve-favicon');

const app = express();
const port = 8000;

app.use(favicon(__dirname + '/favicon.ico'));
app.use('/js', express.static('src/js'));
app.use('/css', express.static('src/css'));
app.use('/pkg-web', express.static('pkg-web'));

app.get('/', (req, res) => {
    res.sendFile('index.html', {root: __dirname});
});

app.listen(port, () => {
    console.log(`Now listening on port ${port}`);
});