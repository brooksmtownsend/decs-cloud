import React, { Component } from 'react';
import {    
    Col,        
    Row    
  } from 'reactstrap';
import Shard from './Shard';

class ShardList extends Component {
    onUpdate = () => {
		this.setState({});
	}

	componentDidMount() {
		this.props.shards.on('add', this.onUpdate);
		this.props.shards.on('remove', this.onUpdate);
	}

	componentWillUnmount() {
		this.props.shards.off('add', this.onUpdate);
		this.props.shards.off('remove', this.onUpdate);
    }
    
    render() {
		// ResClient Collections are iterables, but not arrays.
		return (                                     
          <ul>
            {this.props.shards && Array.from(this.props.shards).map( shard =>
              <Shard key={shard.name} shards={this.props.shards} shard={shard} />
            )}                      
          </ul>
                                
        );
  }
}

export default ShardList;