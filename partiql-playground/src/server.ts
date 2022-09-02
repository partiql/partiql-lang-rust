const express = require('express');
const bodyParser = require("body-parser");
const favicon = require('serve-favicon');
const router = express.Router();
const partiql = require('../pkg-node/partiql_playground');

const app = express();
const port = 8000;


app.use(bodyParser.urlencoded({ extended: false }));
app.use(bodyParser.json());

app.use(favicon(__dirname + '/favicon.ico'));
app.use('/js', express.static('src/js'));
app.use('/css', express.static('src/css'));
app.use('/ace-builds', express.static('src/ace-builds'));
app.use('/pkg-web', express.static('pkg-web'));

app.get('/', (req, res) => {
    res.sendFile('index.html', {root: __dirname});
});

router.post('/parse', (req, res) => {
    res.status(200).json(JSON.stringify(JSON.parse(partiql.parse(req.body.query))));
});

app.use("/", router);

app.listen(port, () => {
    console.log(`Now listening on port ${port}`);
});