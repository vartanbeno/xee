<?php

$queries = new Queries();
$q = $queries->sequence("/root/a/string()");


$documents = new Documents();
$doc = $documents->addString("http://example.com", "<root><a>1</a><a>2</a><a>3</a></root>");

$session = $documents->session();

$seq = $q->execute($session, $doc);

var_dump($seq->len());
