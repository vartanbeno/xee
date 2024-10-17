<?php

$queries = new Xee\Queries();
$q = $queries->sequence('/root/a/string()');
$q_one = $queries->one('/root/a[1]/string()');

$documents = new Xee\Documents();
$doc = $documents->addString("http://example.com", "<root><a>1</a><a>2</a><a>3</a></root>");

var_dump("Sequence");

$seq = $q->execute($documents, $doc);

foreach ($seq as $key => $value) {
    var_dump($key, $value);
    echo "\n";
}

var_dump("One");

$one = $q_one->execute($documents, $doc);
var_dump($one);
