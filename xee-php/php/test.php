<?php

$queries = new Xee\Queries();
$q = $queries->sequence("/root/a/number()");


$documents = new Xee\Documents();
$doc = $documents->addString("http://example.com", "<root><a>1</a><a>2</a><a>3</a></root>");

$session = $documents->session();

// $seq = new XeeSequence();

$seq = $q->execute($session, $doc);

$item = $seq[1];
var_dump($item);
// var_dump($seq[0]);
