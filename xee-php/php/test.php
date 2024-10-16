<?php

$queries = new Xee\Queries();
$q = $queries->sequence('/root/a/string()');

$documents = new Xee\Documents();
$doc = $documents->addString("http://example.com", "<root><a>1</a><a>2</a><a>3</a></root>");


$seq = $q->execute($documents, $doc);

var_dump("Got sequence");

foreach ($seq as $key => $value) {
    var_dump($key, $value);
    echo "\n";
}

var_dump("Foreach end");
