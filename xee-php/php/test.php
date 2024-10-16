<?php

$queries = new Xee\Queries();
$q = $queries->sequence("/root/a/string()");


$documents = new Xee\Documents();
$doc = $documents->addString("http://example.com", "<root><a>1</a><a>2</a><a>3</a></root>");

$session = $documents->session();

// $seq = new XeeSequence();

$seq = $q->execute($session, $doc);


var_dump(count($seq));
// var_dump($seq[0]);
