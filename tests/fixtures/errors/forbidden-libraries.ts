// Test file for forbidden and deprecated libraries

// Error: jQuery is forbidden
import $ from 'jquery';
import jQuery from 'jquery';

// Error: Lodash is forbidden
import _ from 'lodash';
import fp from 'lodash/fp';
import debounce from 'lodash/debounce';
import throttle from 'lodash/throttle';

// Error: Underscore is forbidden
import underscore from 'underscore';

// Error: RxJS is forbidden  
import { Observable } from 'rxjs';
import { Subject } from 'rxjs';

// Error: Minimist has better alternative (node:util parseArgs)
import minimist from 'minimist';

// Error: Yargs has better alternative (node:util parseArgs)
import yargs from 'yargs';

// OK: These libraries are allowed
import React from 'react';
import { useState } from 'react';
import express from 'express';
import axios from 'axios';
import { parseArgs } from 'node:util';

// Error: Forbidden libraries via require (also error for using require)
const lodash = require('lodash'); // 2 errors: require + forbidden
const jquery = require('jquery'); // 2 errors: require + forbidden
const minimistLib = require('minimist'); // 2 errors: require + alternative

// OK: Alternative approach
import { parseArgs as parse } from 'node:util';
const args = parse({ 
  options: {
    verbose: { type: 'boolean' }
  }
});